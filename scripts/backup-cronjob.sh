#!/bin/bash
# scripts/backup-cronjob.sh
# Script de backup automatizado para PostgreSQL (WormGraph) e Redis
#
# Selo: CATHEDRAL-ARKHE-8000-BACKUP-v2.1.0-2026-06-19

set -e

# Configurações
BACKUP_DIR="/mnt/persist/backups"
RETENTION_DAYS=30
DATE_FORMAT=$(date +'%Y-%m-%d_%H-%M-%S')

POSTGRES_CONTAINER="cathedral-postgres"
POSTGRES_USER="cathedral"
POSTGRES_DB="cathedral"

REDIS_CONTAINER="cathedral-redis"
REDIS_DATA_DIR="/var/lib/docker/volumes/cathedral-arkhe_redis-data/_data" # Adjust if named differently

# Cores para o log
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log() {
    echo -e "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

# 1. Garantir diretório de backup
mkdir -p "$BACKUP_DIR/postgres"
mkdir -p "$BACKUP_DIR/redis"

# 2. Backup PostgreSQL
log "${YELLOW}Iniciando backup do PostgreSQL...${NC}"
POSTGRES_BACKUP_FILE="$BACKUP_DIR/postgres/backup_${DATE_FORMAT}.sql.gz"

if docker ps | grep -q "$POSTGRES_CONTAINER"; then
    docker exec "$POSTGRES_CONTAINER" pg_dump -U "$POSTGRES_USER" "$POSTGRES_DB" | gzip > "$POSTGRES_BACKUP_FILE"
    log "${GREEN}Backup PostgreSQL concluído: $POSTGRES_BACKUP_FILE${NC}"
else
    log "${RED}ERRO: Container PostgreSQL ($POSTGRES_CONTAINER) não está rodando.${NC}"
    exit 1
fi

# 3. Backup Redis
log "${YELLOW}Iniciando backup do Redis...${NC}"
REDIS_BACKUP_FILE="$BACKUP_DIR/redis/dump_${DATE_FORMAT}.rdb"

if docker ps | grep -q "$REDIS_CONTAINER"; then
    # Força um save no Redis (Síncrono ou Assíncrono via BGSAVE seria melhor em prod, mas SAVE garante a consistência para o cp)
    docker exec "$REDIS_CONTAINER" redis-cli SAVE

    # Em containers, a melhor forma de pegar o dump.rdb pode variar. Assumindo que o dir de dados é montado
    # Se não tiver acesso ao volume, uma forma alternativa seria ler o rdb via comando docker cp se possível.
    # Vamos usar docker cp para ser mais seguro sobre permissões.
    docker cp "$REDIS_CONTAINER":/data/dump.rdb "$REDIS_BACKUP_FILE"

    log "${GREEN}Backup Redis concluído: $REDIS_BACKUP_FILE${NC}"
else
    log "${RED}ERRO: Container Redis ($REDIS_CONTAINER) não está rodando.${NC}"
fi

# 4. Limpeza de backups antigos
log "${YELLOW}Limpando backups com mais de $RETENTION_DAYS dias...${NC}"
find "$BACKUP_DIR/postgres" -type f -name "*.sql.gz" -mtime +$RETENTION_DAYS -exec rm -f {} \;
find "$BACKUP_DIR/redis" -type f -name "*.rdb" -mtime +$RETENTION_DAYS -exec rm -f {} \;
log "${GREEN}Limpeza concluída.${NC}"

log "${GREEN}Todos os backups finalizados com sucesso.${NC}"
