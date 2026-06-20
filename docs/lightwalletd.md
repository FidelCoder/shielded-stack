# Lightwalletd Notes

`lightwalletd` provides a compact block service used by Zcash light wallets.

## Primary References

- Zcash light client support: https://zcash.readthedocs.io/en/latest/rtd_pages/lightclient_support.html
- Lightwalletd setup: https://zcash.readthedocs.io/en/latest/rtd_pages/lightwalletd.html
- Lightwalletd repository: https://github.com/zcash/lightwalletd
- Light wallet protocol protobufs: https://github.com/zcash/lightwallet-protocol
- ZIP 307: https://zips.z.cash/zip-0307
- Wallet threat model: https://zcash.readthedocs.io/en/latest/rtd_pages/wallet_threat_model.html

## Local Operator Checklist

- Confirm the backing `zcashd` node is synced.
- Confirm `lightwalletd` reports a recent block height.
- Probe the public endpoint from outside the host network.
- Monitor endpoint latency and error rates.
- Keep endpoint metadata current in the operations repository.

