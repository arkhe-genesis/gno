// sphincs_keygen.cpp - Geração de chave SPHINCS+ C13
// Compilar: g++ -std=c++17 -O3 -I/usr/include/openssl sphincs_keygen.cpp -lcrypto -o sphincs_keygen

#include <iostream>
#include <fstream>
#include <iomanip>
#include <cstdint>
#include <cstring>
#include <openssl/rand.h>
#include <openssl/evp.h>
#include <vector>

constexpr size_t N = 16;
constexpr size_t H_PER_LAYER = 12;
constexpr size_t LEAVES_PER_SUBTREE = 1 << H_PER_LAYER; // 4096
constexpr size_t WOTS_MAX_STEP = 7;
constexpr size_t L = 43;

// Funções hash (mesmas do verificador)
void keccak256(const uint8_t* input, size_t in_len, uint8_t* output) {
    EVP_MD_CTX* ctx = EVP_MD_CTX_new();
    EVP_DigestInit_ex(ctx, EVP_sha3_256(), nullptr);
    EVP_DigestUpdate(ctx, input, in_len);
    unsigned int len = 32;
    EVP_DigestFinal_ex(ctx, output, &len);
    EVP_MD_CTX_free(ctx);
    // truncamento para 16 bytes (só nos interessa os primeiros)
}

void chain(const uint8_t* start, size_t steps, uint8_t* out) {
    memcpy(out, start, N);
    for (size_t i = 0; i < steps; ++i) {
        keccak256(out, N, out);
    }
}

void wots_public_key(const uint8_t* seed, uint32_t leaf_idx, uint32_t tree_idx, uint8_t* out) {
    uint8_t tops[L * N];
    for (uint32_t i = 0; i < L; ++i) {
        uint8_t chain_seed[N];
        // Derivação determinística: seed || leaf_idx || tree_idx || i
        uint8_t data[N + 4 + 4 + 2];
        memcpy(data, seed, N);
        memcpy(data + N, &leaf_idx, 4);
        memcpy(data + N + 4, &tree_idx, 4);
        memcpy(data + N + 8, &i, 2);
        keccak256(data, N + 10, chain_seed);
        chain(chain_seed, WOTS_MAX_STEP, tops + i * N);
    }
    // Hash da concatenação dos tops
    keccak256(tops, L * N, out);
}

void merkle_root(const uint8_t** leaves, size_t leaf_count, uint8_t* out) {
    // Encontra a próxima potência de 2
    size_t n = 1;
    while (n < leaf_count) n <<= 1;
    std::vector<uint8_t*> level;
    for (size_t i = 0; i < n; ++i) {
        uint8_t* node = new uint8_t[N];
        if (i < leaf_count) memcpy(node, leaves[i], N);
        else memset(node, 0, N);
        level.push_back(node);
    }
    while (level.size() > 1) {
        std::vector<uint8_t*> next_level;
        for (size_t i = 0; i < level.size(); i += 2) {
            uint8_t* combined = new uint8_t[2 * N];
            memcpy(combined, level[i], N);
            memcpy(combined + N, level[i+1], N);
            uint8_t* parent = new uint8_t[N];
            keccak256(combined, 2 * N, parent);
            next_level.push_back(parent);
            delete[] combined;
        }
        for (auto node : level) delete[] node;
        level = next_level;
    }
    memcpy(out, level[0], N);
    delete[] level[0];
}

void generate_key(uint8_t* secret_seed, uint8_t* public_root) {
    // 1. Seed aleatória (16 bytes) – idealmente via RDRAND ou /dev/urandom
    RAND_bytes(secret_seed, N);

    // 2. Geração das subárvores da camada inferior
    std::vector<uint8_t*> subtree_roots(LEAVES_PER_SUBTREE);
    for (size_t tree_idx = 0; tree_idx < LEAVES_PER_SUBTREE; ++tree_idx) {
        // Para esta subárvore, calculamos as folhas (chaves WOTS+)
        std::vector<uint8_t*> leaves(LEAVES_PER_SUBTREE);
        for (size_t leaf_idx = 0; leaf_idx < LEAVES_PER_SUBTREE; ++leaf_idx) {
            leaves[leaf_idx] = new uint8_t[N];
            wots_public_key(secret_seed, leaf_idx, tree_idx, leaves[leaf_idx]);
        }
        // Raiz da subárvore
        subtree_roots[tree_idx] = new uint8_t[N];
        merkle_root((const uint8_t**)leaves.data(), LEAVES_PER_SUBTREE, subtree_roots[tree_idx]);
        for (auto leaf : leaves) delete[] leaf;
        if ((tree_idx + 1) % 512 == 0) {
            std::cerr << "Subárvores processadas: " << (tree_idx+1) << "/" << LEAVES_PER_SUBTREE << std::endl;
        }
    }
    // 3. Raiz da camada superior (árvore de raízes das subárvores)
    merkle_root((const uint8_t**)subtree_roots.data(), LEAVES_PER_SUBTREE, public_root);
    for (auto root : subtree_roots) delete[] root;
}

int main() {
    uint8_t secret_seed[N];
    uint8_t public_root[N];
    generate_key(secret_seed, public_root);

    std::cout << "GENERATEKEY = AGI – identidade gerada." << std::endl;
    std::cout << "Secret seed (nunca compartilhar): ";
    for (int i = 0; i < N; ++i) std::cout << std::hex << std::setw(2) << std::setfill('0') << (int)secret_seed[i];
    std::cout << std::endl;
    std::cout << "Public key root (registrar na RBB Chain): ";
    for (int i = 0; i < N; ++i) std::cout << std::hex << std::setw(2) << std::setfill('0') << (int)public_root[i];
    std::cout << std::endl;

    // Opcional: salvar em arquivo
    std::ofstream keyfile("cathedral_key.bin", std::ios::binary);
    keyfile.write(reinterpret_cast<char*>(secret_seed), N);
    keyfile.write(reinterpret_cast<char*>(public_root), N);
    keyfile.close();
    return 0;
}
