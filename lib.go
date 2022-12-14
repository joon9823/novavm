package nova

import (
	"encoding/json"

	"github.com/Kernel-Labs/novavm/api"
	"github.com/Kernel-Labs/novavm/types"
)

// VM struct is the core of novavm.
type VM struct {
	inner      api.VM
	printDebug bool
}

// NewVm return VM instance
func NewVM(printDebug bool) VM {
	inner := api.AllocateVM()
	return VM{inner, printDebug}
}

// Initialize deploys std libs and move libs
// for bootstrapping genesis
func (vm *VM) Initialize(
	kvStore api.KVStore,
	moduleBundle types.ModuleBundle,
) error {
	bz, err := json.Marshal(moduleBundle)
	if err != nil {
		return err
	}

	err = api.Initialize(
		vm.inner,
		kvStore,
		vm.printDebug,
		bz,
	)

	return err
}

// VM Destroyer
func (vm *VM) Destroy() {
	api.ReleaseVM(vm.inner)
}

// PublishModuleBundle will publish a given module.
func (vm *VM) PublishModuleBundle(
	kvStore api.KVStore,
	gasLimit uint64,
	txHash types.Bytes, // txHash is used for sessionID
	sender types.AccountAddress,
	moduleBundle types.ModuleBundle,
) (uint64, []types.Event, []types.SizeDelta, error) {
	bz, err := json.Marshal(moduleBundle)
	if err != nil {
		return 0, nil, nil, err
	}

	res, err := api.PublishModuleBundle(
		vm.inner,
		kvStore,
		vm.printDebug,
		gasLimit,
		txHash,
		sender,
		bz,
	)
	if err != nil {
		return 0, nil, nil, err
	}

	var execRes types.ExecutionResult
	err = json.Unmarshal(res, &execRes)

	return execRes.GasUsed, execRes.Events, execRes.SizeDeltas, err
}

// Query will do a query request to VM
func (vm *VM) QueryEntryFunction(
	kvStore api.KVStore,
	goApi api.GoAPI,
	gasLimit uint64,
	payload types.ExecuteEntryFunctionPayload,
) ([]byte, error) {
	bz, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}

	res, err := api.QueryContract(
		vm.inner,
		kvStore,
		goApi,
		vm.printDebug,
		gasLimit,
		bz,
	)

	if err != nil {
		return nil, err
	}

	var execRes types.ExecutionResult
	err = json.Unmarshal(res, &execRes)

	return execRes.Result, err
}

// Execute calls a given contract.
// TODO: add params and returns
func (vm *VM) ExecuteEntryFunction(
	kvStore api.KVStore,
	goApi api.GoAPI,
	gasLimit uint64,
	txHash types.Bytes, // txHash is used for sessionID
	sender types.AccountAddress,
	payload types.ExecuteEntryFunctionPayload,
) (uint64, []types.Event, []types.SizeDelta, error) {
	bz, err := json.Marshal(payload)
	if err != nil {
		return 0, nil, nil, err
	}

	res, err := api.ExecuteContract(
		vm.inner,
		kvStore,
		goApi,
		vm.printDebug,
		gasLimit,
		txHash,
		sender,
		bz,
	)

	if err != nil {
		return 0, nil, nil, err
	}

	var execRes types.ExecutionResult
	err = json.Unmarshal(res, &execRes)
	return execRes.GasUsed, execRes.Events, execRes.SizeDeltas, err
}

// Execute calls a given contract.
// TODO: add params and returns
func (vm *VM) ExecuteScript(
	kvStore api.KVStore,
	goApi api.GoAPI,
	gasLimit uint64,
	txHash types.Bytes, // txHash is used for sessionID
	sender types.AccountAddress,
	payload types.ExecuteScriptPayload,
) (uint64, []types.Event, []types.SizeDelta, error) {
	bz, err := json.Marshal(payload)
	if err != nil {
		return 0, nil, nil, err
	}

	res, err := api.ExecuteScript(
		vm.inner,
		kvStore,
		goApi,
		vm.printDebug,
		gasLimit,
		txHash,
		sender,
		bz,
	)

	if err != nil {
		return 0, nil, nil, err
	}

	var execRes types.ExecutionResult
	err = json.Unmarshal(res, &execRes)
	return execRes.GasUsed, execRes.Events, execRes.SizeDeltas, err
}

// DecodeMoveResource decode resource bytes to move resource
// instance and return as jSON string
func (vm *VM) DecodeMoveResource(
	kvStore api.KVStore,
	structTag string,
	resourceBytes []byte,
) ([]byte, error) {
	return api.DecodeMoveResource(
		kvStore,
		structTag,
		resourceBytes,
	)
}

// DecodeModuleBytes decode module bytes to MoveModule
// instance and return as jSON string
func (vm *VM) DecodeModuleBytes(
	moduleBytes []byte,
) ([]byte, error) {
	return api.DecodeModuleBytes(
		moduleBytes,
	)
}

// DecodeScriptBytes decode script bytes to MoveFunction
// instance and return as jSON string
func (vm *VM) DecodeScriptBytes(
	scriptBytes []byte,
) ([]byte, error) {
	return api.DecodeScriptBytes(
		scriptBytes,
	)
}
