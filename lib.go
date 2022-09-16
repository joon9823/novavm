package kernel

import (
	"github.com/Kernel-Labs/kernelvm/api"
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
	gasMeter api.GasMeter,
	printDebug bool,
	moduleBundle []byte,
) (VM, error) {
	_, err := api.Initialize(kvStore, goApi, querier, gasMeter, printDebug, moduleBundle)

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
	gasMeter api.GasMeter,
	gasLimit uint64,
	sender string,
	message []byte,
) (uint64, error) {
	_, usedGas, err := api.PublishModule(kvStore, goApi, querier, gasMeter, vm.printDebug, gasLimit, sender, message)
	return usedGas, err
}

// Query will do a query request to VM
// TODO: add params and returns
func (vm *VM) Query(
	kvStore api.KVStore,
	goApi api.GoAPI,
	querier api.Querier,
	gasMeter api.GasMeter,
	gasLimit uint64,
	sender string,
	message []byte,
) ([]byte, uint64, error) {
	res, usedGas, err := api.QueryContract(kvStore, goApi, querier, gasMeter, vm.printDebug, gasLimit, sender, message)
	return res, usedGas, err
}

// Execute calls a given contract.
// TODO: add params and returns
func (vm *VM) Execute(
	kvStore api.KVStore,
	goApi api.GoAPI,
	querier api.Querier,
	gasMeter api.GasMeter,
	gasLimit uint64,
	sender string,
	message []byte,
) (uint64, error) {
	_, usedGas, err := api.ExecuteContract(kvStore, goApi, querier, gasMeter, vm.printDebug, gasLimit, sender, message)
	return usedGas, err
}
