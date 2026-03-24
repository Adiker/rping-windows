# rping

[English](#english) | [Polski](#polski)

---

## English

Simple `ping` utility for Windows, written in Rust.

It sends ICMP Echo Request packets via native Windows API (`IcmpSendEcho`) and works without administrator privileges.

### Features

- IPv4 ping by hostname or IP address,
- shows `time`, `TTL`, and response size,
- prints final ping statistics,
- handles `Ctrl+C` (with `-t`),
- configurable interval between requests,
- optional extra session summary (`-S`).

### Build

```bash
cargo build --release
```

### Usage

```bash
rping [-n count] [-w timeout] [-l size] [-i interval] [-S] [-t] <host>
```

### Options

| Flag | Description | Default |
|---|---|---|
| `-n` | number of requests | `4` |
| `-w` | reply timeout (ms) | `4000` |
| `-l` | payload size (bytes) | `32` |
| `-i` | interval between requests (ms) | `1000` |
| `-S` | extra session summary (duration, packets sent, pkt/s) | disabled |
| `-t` | ping continuously (until `Ctrl+C`) | disabled |

### Examples

```bash
rping google.com
rping -n 10 -w 1000 1.1.1.1
rping -n 5 -i 250 -S 8.8.8.8
rping -t -i 500 192.168.1.1
```

[⬆ Back to language switch](#rping)

---

## Polski

Prosty `ping` dla Windows napisany w Rust.

Program wysyła pakiety ICMP Echo Request przez natywne API Windows (`IcmpSendEcho`) i działa bez uprawnień administratora.

### Funkcje

- ping IPv4 po nazwie hosta lub adresie IP,
- pokazuje `time`, `TTL` i rozmiar odpowiedzi,
- wyświetla statystyki po zakończeniu,
- obsługuje `Ctrl+C` (przy `-t`),
- pozwala ustawić interwał między pingami,
- opcjonalnie pokazuje dodatkowe podsumowanie sesji (`-S`).

### Kompilacja

```bash
cargo build --release
```

### Użycie

```bash
rping [-n count] [-w timeout] [-l size] [-i interval] [-S] [-t] <host>
```

### Opcje

| Flaga | Opis | Domyślnie |
|---|---|---|
| `-n` | liczba pakietów | `4` |
| `-w` | timeout odpowiedzi (ms) | `4000` |
| `-l` | rozmiar danych (bajty) | `32` |
| `-i` | interwał między pingami (ms) | `1000` |
| `-S` | dodatkowe podsumowanie sesji (czas, liczba pakietów, pkt/s) | wyłączone |
| `-t` | pingowanie bez końca (do `Ctrl+C`) | wyłączone |

### Przykłady

```bash
rping google.com
rping -n 10 -w 1000 1.1.1.1
rping -n 5 -i 250 -S 8.8.8.8
rping -t -i 500 192.168.1.1
```

[⬆ Wróć do wyboru języka](#rping)
