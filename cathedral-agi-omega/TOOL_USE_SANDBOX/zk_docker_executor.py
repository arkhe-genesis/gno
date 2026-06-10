import docker
import hashlib

class ZKDockerExecutor:
    def run_untrusted_code(self, python_code: str) -> tuple[bytes, str]:
        # Cria um container Docker absolutamente isolado (sem rede, sem IP)
        client = docker.from_env()
        container = client.containers.run(
            "cathedral-sandbox:latest",
            detach=True,
            network_mode="none", # BLINDADO
            mem_limit="512m",
            command=["python", "-c", python_code]
        )

        # Aguarda a execução e captura os logs
        exit_status = container.wait()
        stdout = container.logs().decode('utf-8')

        # Gera um ZK-SNARK provando que o script rodou de forma isolada
        execution_proof = self._generate_execution_proof(python_code, stdout)

        container.remove() # Destrói o container imediatamente

        return execution_proof, stdout

    def _generate_execution_proof(self, code: str, output: str) -> bytes:
        # Em produção, isso invocaria o circom `consistency_check.circom`
        mock_proof = hashlib.sha256(f"{code}{output}".encode()).digest()
        return mock_proof
