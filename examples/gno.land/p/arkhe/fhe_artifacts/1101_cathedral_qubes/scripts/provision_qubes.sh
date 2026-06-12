#!/bin/bash
# Script para provisionamento dos Qubes da Cathedral AGI
# Executar em dom0

set -e

echo "Clonando template minimal Fedora..."
qvm-clone fedora-39-minimal cathedral-template

echo "Instalando dependências no template..."
qvm-run -u root cathedral-template "dnf install -y python3 python3-pip rust cargo golang postgresql-server postgresql-contrib"
qvm-run -u root cathedral-template "cargo install blst"
qvm-run -u root cathedral-template "dnf upgrade -y"

echo "Criando agi-core (Cérebro)..."
qvm-create -l red -t cathedral-template agi-core
qvm-prefs agi-core netvm sys-firewall
qvm-prefs agi-core provides_network false
qvm-prefs agi-core memory 4096
qvm-prefs agi-core maxmem 8192
qvm-prefs agi-core vcpus 4

echo "Criando llm-inference (Mente)..."
qvm-create -l black -t cathedral-template llm-inference
qvm-prefs llm-inference netvm none
qvm-prefs llm-inference memory 16384
qvm-prefs llm-inference maxmem 32768
qvm-prefs llm-inference vcpus 8
# qvm-pci attach llm-inference dom0:00:02.0 --persistent # Substituir BDF

echo "Criando knowledge-base (Memória)..."
qvm-create -l black -t cathedral-template knowledge-base
qvm-prefs knowledge-base netvm none
qvm-prefs knowledge-base memory 4096
qvm-prefs knowledge-base maxmem 8192

echo "Criando governance (Consciência)..."
qvm-create -l black -t cathedral-template governance
qvm-prefs governance netvm none
qvm-prefs governance memory 2048
qvm-prefs governance maxmem 4096

echo "Criando crypto-vm..."
qvm-create -l black -t cathedral-template crypto-vm
qvm-prefs crypto-vm netvm none
qvm-prefs crypto-vm memory 2048

echo "Criando VMs de Ação (Músculos)..."
qvm-create -l yellow -t cathedral-template browser-vm
qvm-prefs browser-vm netvm sys-whonix
qvm-prefs browser-vm memory 2048

qvm-create -l yellow -t cathedral-template email-vm
qvm-prefs email-vm netvm sys-firewall
qvm-prefs email-vm memory 2048

qvm-create -l yellow -t cathedral-template code-vm
qvm-prefs code-vm netvm sys-firewall
qvm-prefs code-vm memory 4096

echo "Criando DispVM Template..."
qvm-create -l green -t cathedral-template cathedral-dvm
qvm-prefs cathedral-dvm template_for_dispvms True

echo "Provisionamento concluído!"
