use crate::bridge::TryIntoKotlin;
use crate::env::{AndroidEnv, KotlinClassName};
use crate::jni_ext::JObjectExt;
use jni::objects::JObject;
use jni::JNIEnv;
use stremio_core::runtime::EnvError;

impl<'a> TryIntoKotlin<'a, ()> for EnvError {
    #[inline]
    fn try_into_kotlin(&self, _args: &(), env: &JNIEnv<'a>) -> jni::errors::Result<JObject<'a>> {
        let classes = AndroidEnv::kotlin_classes().unwrap();
        let code = self.code() as i64;
        let message = self.message().try_into_kotlin(&(), env)?.auto_local(env);
        env.new_object(
            classes.get(&KotlinClassName::EnvError).unwrap(),
            "(ILjava/lang/String;)V",
            &[code.into(), message.as_obj().into()],
        )
    }
}