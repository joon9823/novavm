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
) error {
	_, err := api.Initialize(kvStore, goApi, querier, gasMeter, printDebug, moduleBundle)
	return err
}

// VM Destroyer
// TODO: add params and returns
func (vm *VM) Destroy() {}

// PublishModule will publish a given module.
// TODO: add params and returns
func (vm *VM) PublishModule() {}

// Query will do a query request to VM
// TODO: add params and returns
func (vm *VM) Query() {}

// Execute calls a given contract.
// TODO: add params and returns
func (vm *VM) Execute() {}
