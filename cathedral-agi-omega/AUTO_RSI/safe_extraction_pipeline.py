import subprocess
import hashlib

class SafeExtractionPipeline:
    def generate_and_verify(self, llm_proposed_code: str, target_module: str) -> bool:
        # 1. O LLM gera uma versão otimizada do código
        with open(f"temp/{target_module}_v2.py", "w") as f:
            f.write(llm_proposed_code)

        # 2. Chama o Hax toolchain para extrair o código em C
        try:
            subprocess.run(["hax", "extract", f"temp/{target_module}_v2.py", "-o", "ExtractedSafeCode/"], check=True)
        except subprocess.CalledProcessError:
            return False

        # 3. Executa as provas Lean 4 no código extraído
        try:
            subprocess.run(["lake", "exe", "LEAN4_SUPEREGO/AutoRsiSafety.lean"], check=True)
        except subprocess.CalledProcessError:
            return False

        # 4. Se não estourar, compila o binário e o substitui
        subprocess.run(["gcc", "-O2", "ExtractedSafeCode/*.c", "-o", f"bin/{target_module}_v2.so"], check=True)
        return True
