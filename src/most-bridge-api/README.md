### Most bridge api

### Setup

# Local setup

- register domain in (/etc/hosts) file:
  `::1 local.sufinity`
- host has to be Ipv6 compatible because ICP canisters talk only with Ipv6 hosts
- Create SSL certificates locally with https://github.com/FiloSottile/mkcert

```bash
    mkcert -key-file key.pem -cert-file cert.pem ::1  local.sufinity
```
