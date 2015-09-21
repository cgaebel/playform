use std::marker::PhantomData;

use ::{Flatten, MemStream, EOF, encode, decode};

#[derive(Clone)]
pub struct T<X> {
  data: Vec<u8>,
  phantom: PhantomData<X>,
}

impl<X> Flatten for T<X> {
  fn emit(v: &T<X>, dest: &mut Vec<u8>) -> Result<(), ()> {
    Flatten::emit(&v.data, dest)
  }

  fn read<'a>(s: &mut MemStream<'a>) -> Result<T<X>, EOF> {
    Flatten::read(s)
      .map(|data| {
        T {
          data: data,
          phantom: PhantomData,
        }
      })
  }
}

pub fn new<X>(value: &X) -> T<X>
  where X: Flatten
{
  T {
    data: encode(value).unwrap(),
    phantom: PhantomData,
  }
}

pub fn force<X>(this: &T<X>) -> X
  where X: Flatten
{
  decode(this.data.as_slice()).unwrap()
}
