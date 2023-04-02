define i64 @main() {
  ; allocating 'v'
  %v.0 = alloca i64
  %_st.1 = sub i64 99, 3
  %_dt.1 = sdiv i64 %_st.1, 3
  %_at.1 = add i64 %_dt.1, 3
  %_mt.1 = mul i64 5, 3
  %_at.3 = add i64 32, %_mt.1
  %_st.3 = sub i64 %_at.3, 2
  %_dt.2 = sdiv i64 %_st.3, 9
  %_at.2 = add i64 1, %_dt.2
  %_dt.3 = sdiv i64 100, 25
  %_st.4 = sub i64 %_dt.3, 3
  %_st.2 = sub i64 %_at.2, %_st.4
  %_dt.0 = sdiv i64 %_at.1, %_st.2
  %_mt.2 = mul i64 2, 5
  %_st.5 = sub i64 9, %_mt.2
  %_dt.4 = sdiv i64 8, 4
  %_mt.3 = mul i64 %_dt.4, 2
  %_at.4 = add i64 %_st.5, %_mt.3
  %_mt.0 = mul i64 %_dt.0, %_at.4
  %_st.0 = sub i64 %_mt.0, 15
  %_at.0 = add i64 %_st.0, 1
  store i64 %_at.0, ptr %v.0
  ; dereferencing 'v'
  %v.1 = load i64, ptr %v.0
  ret i64 %v.1
}

