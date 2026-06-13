## Migrating from Binary

If you're used to binary signal processing, ternary adds a crucial middle state: **noise floor|Positive pulse|Neutral baseline|Negative pulse|DC offset as neutral reference**.

The $0$ (neutral) state is the key difference — it captures "noise floor|Positive pulse|Neutral baseline|Negative pulse|DC offset as neutral reference" rather than forcing everything into a binary choice.

See **[From Binary to Ternary](https://github.com/SuperInstance/ternary-cookbook/blob/master/guides/FROM_BINARY.md)** for the full migration guide.
