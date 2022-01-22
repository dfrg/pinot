pub fn read_n<'a, T>(buf: &'a [u8], offset: usize, n: usize) -> Option<&'a [T]> {
    let end = offset.checked_add(n.checked_mul(core::mem::size_of::<T>())?)?;
    let bytes = buf.get(offset..end)?;
    Some(unsafe { core::slice::from_raw_parts(bytes.as_ptr() as _, n) })
}
