package tech.turso.libsql

import kotlin.reflect.KProperty

class Pointer(private var inner: Long) {
    operator fun getValue(thisRef: Any?, property: KProperty<*>): Long {
        require(this.inner != 0L) { "Attempted to use a null pointer" }
        return this.inner
    }

    operator fun setValue(thisRef: Any?, property: KProperty<*>, value: Long) {
        this.inner = value
    }
}