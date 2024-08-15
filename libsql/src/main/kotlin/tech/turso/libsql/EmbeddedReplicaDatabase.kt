package tech.turso.libsql

class EmbeddedReplicaDatabase internal constructor(inner: Long) : Database(inner) {
    fun sync() {
        nativeSync(this.inner)
    }

    private external fun nativeSync(db: Long)
}
