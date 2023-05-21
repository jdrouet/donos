# Donos

An attempt to rewrite something to [Pi-Hole](https://pi-hole.net/), fully in [Rust](https://www.rust-lang.org/), without a UI (for now).

> ❗️This is under development, some things might not work properly.

## Why Donos?

Dealing with UIs, clicking buttons, could be cumbersome. What if we could just create a configuration file, simple to understand and then start the DNS server and it just works?

Also, why would we need to have a HTTP server running 100% of the time if we only use it once a week?

Personnal security is a serious topic. Global warming is one as well. We should build softwares that are compatible with those two subjects. We should make safe, secure and reliable software but also with the smallest footprint as possible. Splitting the whole [Pi-Hole](https://pi-hole.net/) application into separate services that we could run independently is one way to start.

## How does it work?

> TODO

### Features

- [ ] loading blocklist in the database
    - [x] loading /etc/hosts format
    - [x] loading no-ip format
    - [ ] loading dnsmasq format
    - [ ] loading adguard format
- [ ] handle dns requests
    - [x] respond as a simple cache
    - [x] block requests for domains in the blocklist
    - [ ] block requests for domains that a device subscribed to
- [ ] observability
    - [ ] being able to follow all the requests through the logs
    - [ ] export usage metrics (similar to [Pi-Hole](https://pi-hole.net/))

> This list is not complete

## Why the name Donos?

I'm not good with names, just put "o" between each letter of "DNS" and you get Donos... 🤷‍♂️

## Thanks

I didn't learn how the DNS protocol works in one night. This is heavily inspired by [dnsguide](https://github.com/EmilHernvall/dnsguide).