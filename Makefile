# TODO: this can be rewritten as build.rs in the sandbox crate,
# but it might introduce unnecessary friction and longer build times.
# Moreover, any project using sandbox as dependency won't build unless
# there's Foundry installed on the machine.

.PHONY: artifacts

artifacts:
	mkdir crates/starknet-proxy-client/src/artifacts || true
	mkdir crates/starknet-core-contract-client/src/artifacts || true
	forge build
	cp out/Proxy.sol/Proxy.json crates/starknet-proxy-client/src/artifacts/
	cp out/StarknetSovereign.sol/Starknet.json crates/starknet-core-contract-client/src/artifacts/