define i64 @main() {
  ; allocating 'a'
  %.1 = alloca i64
  ; 'multiplication' binop
  ; 'multiplication' binop
  %.2 = mul i64 4, 12
  %.3 = mul i64 %.2, 12
  store i64 %.3, ptr %.1
  ; if
  ; dereferencing 'a'
  %.4 = load i64, ptr %.1
  %.5 = icmp slt i64 %.4, 2
  br i1 %.5, label %.6, label %.7
.6:
  ret i64 4
  br label %.8
.7:
  ret i64 3
  br label %.8
.8:
  ret i64 3
  unreachable
}

