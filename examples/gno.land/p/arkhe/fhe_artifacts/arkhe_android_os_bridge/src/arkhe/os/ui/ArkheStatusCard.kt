package arkhe.os.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

data class ArkheStatus(
    val agentId: String = "Unknown",
    val substratesActive: Int = 0,
    val komogorovBits: Long = 0
)

@Composable
fun arkheStatusState(): androidx.compose.runtime.State<ArkheStatus> {
    // In a real implementation this would bind to the ArkheSystemService AIDL interface
    return remember { mutableStateOf(ArkheStatus("Catedral ARKHE — Omni", 8, 4096)) }
}

@Composable
fun ArkheStatusCard() {
    val status by arkheStatusState()
    Card(modifier = Modifier.padding(16.dp)) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text("Catedral ARKHE — ${status.agentId}")
            Text("Substratos: ${status.substratesActive}")
            Text("Complexidade K: ${status.komogorovBits} bits")
        }
    }
}
