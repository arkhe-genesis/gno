#!/bin/bash
# install_btfs_in_template.sh – Instala BTFS Go client no cathedral-template

set -e

# Baixar BTFS Go (última versão estável)
BTFS_VERSION="2.17.0"
wget -O /tmp/btfs.tar.gz "https://github.com/bittorrent/go-btfs/releases/download/v${BTFS_VERSION}/btfs-v${BTFS_VERSION}-linux-amd64.tar.gz"
tar -xzf /tmp/btfs.tar.gz -C /usr/local/bin --strip-components=1 btfs/btfs

# Configurar diretório de dados
mkdir -p /var/lib/btfs
chmod 755 /var/lib/btfs

# Inicializar repositório BTFS (executar como usuário normal depois)
# btfs init --p2p-version=1 --server-mode

# Instalar dependências adicionais (se necessário)
dnf install -y fuse

# Limpeza
rm /tmp/btfs.tar.gz
