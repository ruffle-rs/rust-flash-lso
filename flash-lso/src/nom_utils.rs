use cookie_factory::SerializeFn;
use std::io::Write;

pub(crate) fn either<Fa, Fb, W: Write>(b: bool, t: Fa, f: Fb) -> impl SerializeFn<W>
where
    Fa: SerializeFn<W>,
    Fb: SerializeFn<W>,
{
    move |out| {
        if b {
            t(out)
        } else {
            f(out)
        }
    }
}
