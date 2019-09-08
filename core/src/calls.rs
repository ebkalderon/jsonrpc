use std::fmt;
use std::future::Future;
use std::sync::Arc;

use crate::types::{Params, Value};
use crate::{BoxFuture, Result};

/// Metadata trait
pub trait Metadata: Clone + Send + 'static {}
impl Metadata for () {}
impl<T: Metadata> Metadata for Option<T> {}
impl<T: Metadata> Metadata for Box<T> {}
impl<T: Sync + Send + 'static> Metadata for Arc<T> {}

/// Asynchronous Method
pub trait RpcMethodSimple: Send + Sync + 'static {
	/// Output future
	type Out: Future<Output = Result<Value>> + Send + Unpin;
	/// Call method
	fn call(&self, params: Params) -> Self::Out;
}

/// Asynchronous Method with Metadata
pub trait RpcMethod<T: Metadata>: Send + Sync + 'static {
	/// Call method
	fn call(&self, params: Params, meta: T) -> BoxFuture<Value>;
}

/// Notification
pub trait RpcNotificationSimple: Send + Sync + 'static {
	/// Execute notification
	fn execute(&self, params: Params);
}

/// Notification with Metadata
pub trait RpcNotification<T: Metadata>: Send + Sync + 'static {
	/// Execute notification
	fn execute(&self, params: Params, meta: T);
}

/// Possible Remote Procedures with Metadata
#[derive(Clone)]
pub enum RemoteProcedure<T: Metadata> {
	/// A method call
	Method(Arc<dyn RpcMethod<T>>),
	/// A notification
	Notification(Arc<dyn RpcNotification<T>>),
	/// An alias to other method,
	Alias(String),
}

impl<T: Metadata> fmt::Debug for RemoteProcedure<T> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use self::RemoteProcedure::*;
		match *self {
			Method(..) => write!(fmt, "<method>"),
			Notification(..) => write!(fmt, "<notification>"),
			Alias(ref alias) => write!(fmt, "alias => {:?}", alias),
		}
	}
}

impl<F: Send + Sync + 'static, T: Send> RpcMethodSimple for F
where
	F: Fn(Params) -> T,
	T: Future<Output = Result<Value>> + Unpin,
{
	type Out = T;
	fn call(&self, params: Params) -> Self::Out {
		self(params)
	}
}

impl<F: Send + Sync + 'static> RpcNotificationSimple for F
where
	F: Fn(Params),
{
	fn execute(&self, params: Params) {
		self(params)
	}
}

impl<F: Send + Sync + 'static, T: Send, M> RpcMethod<M> for F
where
	M: Metadata,
	F: Fn(Params, M) -> T,
	T: Future<Output = Result<Value>> + Unpin + 'static,
{
	fn call(&self, params: Params, meta: M) -> BoxFuture<Value> {
		Box::new(self(params, meta))
	}
}

impl<F: Send + Sync + 'static, M> RpcNotification<M> for F
where
	M: Metadata,
	F: Fn(Params, M),
{
	fn execute(&self, params: Params, meta: M) {
		self(params, meta)
	}
}
