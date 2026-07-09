 fn main() -> ()  {  let x: bool;  let r: &mut bool;   x =  false;   r =  &mut  x;   * r =  true;   print( x); }
