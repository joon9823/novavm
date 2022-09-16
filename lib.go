package kernel

import (
	"github.com/Kernel-Labs/kernelvm/api"
	"github.com/Kernel-Labs/kernelvm/types"
)

// VM struct is the core of kernelvm.
type VM struct {
	printDebug bool
}

// CreateVM creates a new VM.
// TODO: add params and returns
func CreateVM(
	kvStore api.KVStore,
	goApi api.GoAPI,
	querier api.Querier,
	printDebug bool,
	moduleBundle []byte,
) (VM, error) {
	_, err := api.Initialize(
		kvStore,
		goApi,
		querier,
		printDebug,
		moduleBundle,
	)

	return VM{
		printDebug,
	}, err
}

// VM Destroyer
// TODO: add params and returns
func (vm *VM) Destroy() {}

// PublishModule will publish a given module.
// TODO: add params and returns
func (vm *VM) PublishModule(
	kvStore api.KVStore,
	goApi api.GoAPI,
	querier api.Querier,
	gasLimit uint64,
	sender types.AccountAddress,
	message []byte,
) (uint64, error) {
	_, usedGas, err := api.PublishModule(
		kvStore,
		goApi,
		querier,
		vm.printDebug,
		gasLimit,
		sender,
		message,
	)

	return usedGas, err
}

// Query will do a query request to VM
// TODO: add params and returns
func (vm *VM) Query(
	kvStore api.KVStore,
	goApi api.GoAPI,
	querier api.Querier,
	gasLimit uint64,
	sender types.AccountAddress,
	message []byte,
) ([]byte, uint64, error) {
	return api.QueryContract(
		kvStore,
		goApi,
		querier,
		vm.printDebug,
		gasLimit,
		sender,
		message,
	)
}

// Execute calls a given contract.
// TODO: add params and returns
func (vm *VM) Execute(
	kvStore api.KVStore,
	goApi api.GoAPI,
	querier api.Querier,
	gasLimit uint64,
	sender types.AccountAddress,
	message []byte,
) (uint64, error) {
	_, usedGas, err := api.ExecuteContract(
		kvStore,
		goApi,
		querier,
		vm.printDebug,
		gasLimit,
		sender,
		message,
	)
	return usedGas, err
}
