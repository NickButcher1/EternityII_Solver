package eternityii

object BoardOrder {
    fun getBoardOrder(): List<SearchIndex> {
        val boardOrder = listOf(
            listOf(196, 197, 198, 199, 200, 205, 210, 215, 220, 225, 230, 235, 243, 249, 254, 255),
            listOf(191, 192, 193, 194, 195, 204, 209, 214, 219, 224, 229, 234, 242, 248, 252, 253),
            listOf(186, 187, 188, 189, 190, 203, 208, 213, 218, 223, 228, 233, 241, 247, 250, 251),
            listOf(181, 182, 183, 184, 185, 202, 207, 212, 217, 222, 227, 232, 240, 244, 245, 246),
            listOf(176, 177, 178, 179, 180, 201, 206, 211, 216, 221, 226, 231, 236, 237, 238, 239),
            listOf(160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175),
            listOf(144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159),
            listOf(128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143),
            listOf(112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127),
            listOf(96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111),
            listOf(80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95),
            listOf(64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79),
            listOf(48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63),
            listOf(32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47),
            listOf(16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31),
            listOf(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15)
        )

        val boardSearchSequence: MutableMap<Int, SearchIndex> = mutableMapOf()

        for (row in 0..15) {
            for (col in 0..15) {
                val pieceSequenceNumber = boardOrder[15 - row][col]
                boardSearchSequence[pieceSequenceNumber] = SearchIndex(row.toByte(), col.toByte())
            }
        }

        val boardSearchSequenceList = mutableListOf<SearchIndex>()
        for (i in 0..255) {
            boardSearchSequenceList.add(boardSearchSequence[i]!!)
        }

        return boardSearchSequenceList.toList()
    }
}
