package com.nix.stsms

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import com.nix.stsms.db.StsmsRepository
import com.nix.stsms.ui.StsmsApp
import com.nix.stsms.ui.StsmsController
import java.io.File

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val dbFile = File(filesDir, "stsms_data.db")
        val storageRoot = File(filesDir, "storage_root")
        val repo = StsmsRepository(dbFile = dbFile, storageRootDir = storageRoot)
        val controller = StsmsController(repo = repo)

        setContent {
            StsmsApp(controller = controller)
        }
    }
}
