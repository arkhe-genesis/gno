package arkhe.os.binder;

interface IArkheCryptoService {
    String signMessage(String message);
    String encryptFHE(in float[] vector);
    boolean verifyPQC(String signature, String message);
}
