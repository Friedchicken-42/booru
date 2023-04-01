use futures::{
    future::{try_join_all, BoxFuture},
    FutureExt,
};
use serde::{de::Error, Deserialize, Deserializer};
use std::{future::Future, sync::Arc};

// Waiting for this: https://github.com/serde-rs/serde/pull/2403

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Tagged<T> {
    NOT(Box<Pattern<T>>),
    #[serde(deserialize_with = "check_multiple")]
    AND(Vec<Pattern<T>>),
    #[serde(deserialize_with = "check_multiple")]
    OR(Vec<Pattern<T>>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Pattern<T> {
    Item(T),
    Tagged(Tagged<T>),
}

fn check_multiple<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let v = <Vec<T>>::deserialize(d)?;
    if v.len() >= 2 {
        Ok(v)
    } else {
        Err(D::Error::custom("Need 2 or more elements"))
    }
}

impl<'a, T> Tagged<T>
where
    T: Send + 'a,
{
    fn serialize(self, set: &str) -> String
        where T: ToString
    {
        match self {
            Self::NOT(x) => match *x {
                Pattern::Item(y) => format!("{} notinside {}", y.to_string(), set),
                _ => format!("(({}) == false)", x.serialize(set)),
            },
            Self::AND(v) => Self::join(v, "&&", set),
            Self::OR(v) => Self::join(v, "||", set),
        }
    }

    fn join(patterns: Vec<Pattern<T>>, separator: &str, set: &str) -> String 
        where T: ToString
    {
        let s = patterns
            .into_iter()
            .map(|p| p.serialize(set))
            .reduce(|a, b| format!("{} {} {}", a, separator, b))
            .unwrap();

        format!("({})", s)
    }

    fn convert<U, F, Fut>(self, f: Arc<F>) -> BoxFuture<'a, Result<Tagged<U>, ()>>
    where
        F: Fn(T) -> Fut + Sync + Send + 'a,
        Fut: Future<Output = Result<U, ()>> + Send,
        U: Send,
    {
        async move {
            let p = match self {
                Tagged::NOT(x) => Tagged::NOT(Box::new(x.convert(f).await?)),
                Tagged::AND(v) => Tagged::AND(Self::convert_iter(v, f).await?),
                Tagged::OR(v) => Tagged::OR(Self::convert_iter(v, f).await?),
            };

            Ok(p)
        }
        .boxed()
    }

    async fn convert_iter<F, Fut, U>(v: Vec<Pattern<T>>, f: Arc<F>) -> Result<Vec<Pattern<U>>, ()>
    where
        F: Fn(T) -> Fut + Sync + Send + 'a,
        Fut: Future<Output = Result<U, ()>> + Send,
        U: Send,
    {
        try_join_all(v.into_iter().map(|t| t.convert(Arc::clone(&f)))).await
    }
}

impl<'a, T> Pattern<T>
where
    T: Send + 'a,
{
    pub fn serialize(self, set: &str) -> String 
        where T: ToString
    {
        match self {
            Self::Item(x) => format!("{} inside {}", x.to_string(), set),
            Self::Tagged(x) => x.serialize(set),
        }
    }

    pub fn convert<U, F, Fut>(self, f: Arc<F>) -> BoxFuture<'a, Result<Pattern<U>, ()>>
    where
        F: Fn(T) -> Fut + Sync + Send + 'a,
        Fut: Future<Output = Result<U, ()>> + Send,
        U: Send,
    {
        async move {
            let p = match self {
                Pattern::Item(x) => Pattern::Item(f(x).await?),
                Pattern::Tagged(x) => Pattern::Tagged(x.convert(f).await?),
            };

            Ok(p)
        }
        .boxed()
    }
}
