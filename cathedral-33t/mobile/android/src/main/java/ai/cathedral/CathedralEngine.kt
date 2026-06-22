package ai.cathedral

import android.content.Context
import java.io.File

class CathedralEngine(private val context: Context) {
    init {
        System.loadLibrary("cathedral_arkhe")
    }

    external fun loadModel(path: String): Boolean
    external fun infer(input: String): String
    external fun inferFloat(input: FloatArray): FloatArray
    external fun reset()

    fun loadModelFromAssets(filename: String): Boolean {
        return try {
            val file = File(context.filesDir, filename)
            context.assets.open(filename).use { input ->
                file.outputStream().use { output ->
                    input.copyTo(output)
                }
            }
            loadModel(file.absolutePath)
        } catch (e: Exception) {
            false
        }
    }

    companion object {
        init {
            System.loadLibrary("cathedral_arkhe")
        }
    }
}
