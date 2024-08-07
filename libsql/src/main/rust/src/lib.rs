#![allow(non_snake_case)]

use jni::{
    objects::{JByteArray, JClass, JObject, JObjectArray, JString, JValue},
    sys::{jdouble, jlong, jobjectArray},
    JNIEnv,
};
use jni_fn::jni_fn;
use libsql::{Builder, Connection, Database, Rows};
use proto::NamedParameters;
use std::{mem::ManuallyDrop, ptr};

use lazy_static::lazy_static;
use prost::Message;
use tokio::runtime::Runtime;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

impl From<proto::Value> for libsql::Value {
    fn from(value: proto::Value) -> Self {
        use proto::value::Value as V;

        match value.value {
            Some(V::Integer(i)) => libsql::Value::Integer(i),
            Some(V::Real(r)) => libsql::Value::Real(r),
            Some(V::Text(s)) => libsql::Value::Text(s),
            Some(V::Blob(b)) => libsql::Value::Blob(b),
            Some(V::Null(_)) => libsql::Value::Null,
            None => libsql::Value::Null,
        }
    }
}

lazy_static! {
    static ref RT: Runtime = Runtime::new().unwrap();
}

#[jni_fn("tech.turso.libsql.Libsql")]
pub fn nativeOpenLocal(mut env: JNIEnv, _: JClass, path: JString) -> jlong {
    let path = match env.get_string(&path) {
        Ok(path) => path,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return ptr::null_mut::<Database>() as jlong;
        }
    };

    let db = match RT.block_on(Builder::new_local(&*path.to_string_lossy()).build()) {
        Ok(db) => db,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return ptr::null_mut::<Database>() as jlong;
        }
    };

    Box::into_raw(Box::new(db)) as jlong
}

#[jni_fn("tech.turso.libsql.Libsql")]
pub fn nativeOpenRemote(mut env: JNIEnv, _: JClass, url: JString, auth_token: JString) -> jlong {
    let url = match env.get_string(&url) {
        Ok(path) => path,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return ptr::null_mut::<Database>() as jlong;
        }
    };

    let auth_token = match env.get_string(&auth_token) {
        Ok(path) => path,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return ptr::null_mut::<Database>() as jlong;
        }
    };

    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_or_http()
        .enable_http1()
        .build();

    let db = RT.block_on(
        Builder::new_remote(url.into(), auth_token.into())
            .connector(connector)
            .build(),
    );

    match db {
        Ok(db) => Box::into_raw(Box::new(db)) as jlong,
        Err(_) => ptr::null_mut::<Database>() as jlong,
    }
}

#[jni_fn("tech.turso.libsql.Libsql")]
pub fn nativeOpenEmbeddedReplica(
    mut env: JNIEnv,
    _: JClass,
    path: JString,
    url: JString,
    auth_token: JString,
) -> jlong {
    let path = match env.get_string(&path) {
        Ok(path) => path,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return ptr::null_mut::<Database>() as jlong;
        }
    };

    let url = match env.get_string(&url) {
        Ok(path) => path,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return ptr::null_mut::<Database>() as jlong;
        }
    };

    let auth_token = match env.get_string(&auth_token) {
        Ok(path) => path,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return ptr::null_mut::<Database>() as jlong;
        }
    };

    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_or_http()
        .enable_http1()
        .build();

    let db = RT.block_on(
        Builder::new_remote_replica(&*path.to_string_lossy(), url.into(), auth_token.into())
            .connector(connector)
            .build(),
    );

    match db {
        Ok(db) => Box::into_raw(Box::new(db)) as jlong,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            ptr::null_mut::<Database>() as jlong
        }
    }
}

#[jni_fn("tech.turso.libsql.Database")]
pub fn nativeConnect(mut env: JNIEnv, _: JClass, ptr: jlong) -> jlong {
    let db = ManuallyDrop::new(unsafe { Box::from_raw(ptr as *mut Database) });
    match db.connect() {
        Ok(conn) => Box::into_raw(Box::new(conn)) as jlong,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            ptr::null_mut::<Connection>() as jlong
        }
    }
}

#[jni_fn("tech.turso.libsql.Database")]
pub fn nativeClose(_: JNIEnv, _: JClass, ptr: jlong) {
    drop(unsafe { Box::from_raw(ptr as *mut Database) });
}

#[jni_fn("tech.turso.libsql.EmbeddedReplicaDatabase")]
pub fn nativeSync(mut env: JNIEnv, _: JClass, ptr: jlong) {
    let db = ManuallyDrop::new(unsafe { Box::from_raw(ptr as *mut Database) });

    match RT.block_on(db.sync()) {
        Ok(_) => (),
        Err(err) => env.throw(err.to_string()).unwrap(),
    }
}

#[jni_fn("tech.turso.libsql.Connection")]
pub fn nativeExecute(mut env: JNIEnv, _: JClass, ptr: jlong, sql: JString) {
    let sql = match env.get_string(&sql) {
        Ok(path) => path,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return;
        }
    };
    let db = ManuallyDrop::new(unsafe { Box::from_raw(ptr as *mut Connection) });
    RT.block_on(db.execute(&sql.to_string_lossy(), ())).unwrap();
}

#[jni_fn("tech.turso.libsql.Connection")]
pub fn nativeQuery(
    mut env: JNIEnv,
    _: JClass,
    conn: jlong,
    sql: JString,
    buf: JByteArray,
) -> jlong {
    let sql = match env.get_string(&sql) {
        Ok(path) => path,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return ptr::null_mut::<Rows>() as jlong;
        }
    };

    let conn = conn as *mut Connection;
    let conn = ManuallyDrop::new(unsafe { Box::from_raw(conn) });

    let buf = match env.convert_byte_array(buf) {
        Ok(buf) => buf,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return ptr::null_mut::<Rows>() as jlong;
        }
    };

    let params = match proto::Parameters::decode(buf.as_slice()) {
        Ok(params) => params,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            return ptr::null_mut::<Rows>() as jlong;
        }
    };

    let proto::parameters::Params::Named(NamedParameters { parameters: params }) =
        params.params.unwrap()
    else {
        unreachable!()
    };

    let params = params
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect::<Vec<(String, libsql::Value)>>();

    match RT.block_on(conn.query(&sql.to_string_lossy(), params)) {
        Ok(row) => Box::into_raw(Box::new(row)) as jlong,
        Err(err) => {
            env.throw(err.to_string()).unwrap();
            ptr::null_mut::<Rows>() as jlong
        }
    }
}

#[jni_fn("tech.turso.libsql.Connection")]
pub fn nativeClose(_: JNIEnv, _: JClass, ptr: jlong) {
    drop(unsafe { Box::from_raw(ptr as *mut Connection) });
}

#[jni_fn("tech.turso.libsql.Rows")]
pub fn nativeNextRow(mut env: JNIEnv, _: JClass, ptr: jlong) -> jobjectArray {
    let mut rows = ManuallyDrop::new(unsafe { Box::from_raw(ptr as *mut Rows) });
    RT.block_on(async {
        match rows.next().await {
            Ok(Some(row)) => {
                let count = rows.column_count();
                match env.new_object_array(count, "java/lang/Object", JObject::null()) {
                    Ok(arr) => {
                        for i in 0..count {
                            let val = row.get_value(i).unwrap();
                            let obj = match val {
                                libsql::Value::Null => JObject::null(),
                                libsql::Value::Integer(v) => env
                                    .new_object(
                                        "java/lang/Long",
                                        "(J)V",
                                        &[JValue::from(v as jlong)],
                                    )
                                    .unwrap(),
                                libsql::Value::Real(v) => env
                                    .new_object(
                                        "java/lang/Double",
                                        "(D)V",
                                        &[JValue::from(v as jdouble)],
                                    )
                                    .unwrap(),
                                libsql::Value::Text(v) => env.new_string(v).unwrap().into(),
                                libsql::Value::Blob(v) => {
                                    env.byte_array_from_slice(&v).unwrap().into()
                                }
                            };
                            env.set_object_array_element(&arr, i, obj).unwrap();
                        }
                        arr
                    }
                    _ => JObjectArray::default(),
                }
            }
            _ => JObjectArray::default(),
        }
    })
    .into_raw()
}

#[jni_fn("tech.turso.libsql.Rows")]
pub fn nativeClose(_: JNIEnv, _: JClass, ptr: jlong) {
    drop(unsafe { Box::from_raw(ptr as *mut Rows) });
}
