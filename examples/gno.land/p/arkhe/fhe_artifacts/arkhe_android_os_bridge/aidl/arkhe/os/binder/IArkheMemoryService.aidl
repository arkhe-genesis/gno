package arkhe.os.binder;

import android.os.Bundle;
import java.util.List;

interface IArkheMemoryService {
    String commitMemory(in Bundle content, double relevance);
    List<Bundle> retrieveRelevant(String query);
    Bundle getAgentStatus();
}
