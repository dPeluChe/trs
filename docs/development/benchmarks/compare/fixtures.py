"""Three small projects with two failing tests each, so test-runner output is
part of the corpus without depending on any real project's state."""
import os, sys

root = sys.argv[1]
for d in ("cargofail/src", "buntest", "pyfail"):
    os.makedirs(os.path.join(root, d), exist_ok=True)

open(f"{root}/cargofail/Cargo.toml", "w").write(
    '[package]\nname = "cargofail"\nversion = "0.1.0"\nedition = "2021"\n')
rs = ["fn add(a: i32, b: i32) -> i32 { a + b }", "fn main() { println!(\"{}\", add(1, 2)); }",
      "#[cfg(test)] mod tests { use super::*;"]
rs += [f"  #[test] fn passes_{i}() {{ assert_eq!(add({i}, 1), {i + 1}); }}" for i in range(60)]
rs += ['  #[test] fn fails_on_overflow_boundary() { assert_eq!(add(2, 2), 5, "addition drifted"); }',
       '  #[test] fn fails_parsing_config() { "abc".parse::<i32>().unwrap(); }', "}"]
open(f"{root}/cargofail/src/main.rs", "w").write("\n".join(rs) + "\n")

ts = ["import { test, expect } from 'bun:test';"]
ts += [f"test('adds case {i}', () => {{ expect({i}+1).toBe({i + 1}); }});" for i in range(80)]
ts += ["test('formats currency for MXN locale', () => { expect((1234.5).toFixed(1)).toBe('1,234.50'); });",
       "test('rejects expired token', () => { expect({ ok: true, reason: 'valid' }).toEqual({ ok: false, reason: 'expired' }); });"]
open(f"{root}/buntest/math.test.ts", "w").write("\n".join(ts) + "\n")

py = [f"def test_ok_{i}():\n    assert {i} + 1 == {i + 1}\n" for i in range(70)]
py += ["def test_invoice_total_rounding():\n    total = round(0.1 + 0.2, 2)\n    assert total == 0.31, f'expected 0.31, got {total}'\n",
       "def test_user_lookup_missing():\n    users = {'ana': 1}\n    assert users['beto'] == 2\n"]
open(f"{root}/pyfail/test_app.py", "w").write("\n".join(py))
