package tech.turso.libsql

import tech.turso.libsql.proto.Row
import tech.turso.libsql.proto.Value

class Rows internal constructor(inner: Long) : AutoCloseable {
    private var inner by Pointer(inner)

    fun next(): List<Value> = Row.parseFrom(nativeNext(this.inner)).valuesList

    override fun close() {
        nativeClose(this.inner)
        this.inner = 0L
    }

    private external fun nativeNext(rows: Long): ByteArray

    private external fun nativeClose(rows: Long)
}
