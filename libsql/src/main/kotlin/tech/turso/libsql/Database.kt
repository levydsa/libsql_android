package tech.turso.libsql

open class Database internal constructor(inner: Long) : AutoCloseable {
    protected var inner by Pointer(inner)

    fun connect(): Connection {
        return ConnectionImpl(nativeConnect(this.inner))
    }

    override fun close() {
        nativeClose(this.inner)
        this.inner = 0L;
    }

    private external fun nativeClose(db: Long)

    private external fun nativeConnect(db: Long): Long
}
