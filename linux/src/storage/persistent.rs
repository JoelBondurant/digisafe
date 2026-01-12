// State<Unlocked> <-> Avro <-> LZ4 <-> ChaCha20Poly1305 <-> Base64 <-> State<Locked>
// State<Locked> -> Avro -> Reed-Solomon -> Vec<u8> -> name.digisafe
