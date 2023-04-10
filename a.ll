define i64 @add(i64 %a, i64 %b) {
  %.1 = alloca i64
  store i64 %a, ptr %.1
  %.2 = alloca i64
  store i64 %b, ptr %.2
  ; 'addition' binop
  ; dereferencing 'a'
  %.3 = load i64, ptr %.1
  ; dereferencing 'b'
  %.4 = load i64, ptr %.2
  %.5 = add i64 %.3, %.4
  ret i64 %.5
  unreachable
}
define i64 @main() {
  ; allocating 'a'
  %.6 = alloca i64
  %.7 = call i64 @add(i64 8, i64 4)
  store i64 %.7, ptr %.6
  ; dereferencing 'a'
  %.8 = load i64, ptr %.6
  ret i64 %.8
  unreachable
}

