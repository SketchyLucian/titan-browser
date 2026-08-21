package com.titan.browser.storage

import com.titan.browser.model.HistoryEntry
import org.junit.Assert.assertEquals
import org.junit.Test

class HistoryPolicyTest {

    @Test
    fun movesRepeatVisitsToTheFrontAndIncrementsCount() {
        val existing = listOf(
            HistoryEntry("First", "https://first.test", 10, 1),
            HistoryEntry("Old title", "https://repeat.test", 5, 3)
        )

        val updated = StorageManager.updateHistory(
            existing,
            "New title",
            "https://repeat.test",
            20
        )

        assertEquals(2, updated.size)
        assertEquals("https://repeat.test", updated.first().url)
        assertEquals("New title", updated.first().title)
        assertEquals(4, updated.first().visitCount)
        assertEquals(20, updated.first().lastVisitedMs)
    }
}
