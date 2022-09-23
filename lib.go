package nova

import (
	"encoding/json"

	"github.com/Kernel-Labs/novavm/api"
	"github.com/Kernel-Labs/novavm/types"
)

// VM struct is the core of novavm.
type VM struct {
	printDebug bool
}

// NewVm return VM instance
func NewVM(printDebug bool) VM {
	return VM{printDebug}
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

	_, err = api.Initialize(
		kvStore,
		vm.printDebug,
		bz,
	)

	return err
}

// VM Destroyer
// TODO: add params and returns
func (vm *VM) Destroy() {}

// PublishModule will publish a given module.
// TODO: add params and returns
func (vm *VM) PublishModule(
	kvStore api.KVStore,
	gasLimit uint64,
	sender types.AccountAddress,
	moduleBytes []byte,
) (uint64, error) {
	res, err := api.PublishModule(
		kvStore,
		vm.printDebug,
		gasLimit,
		sender,
		moduleBytes,
	)
	if err != nil {
		return 0, err
	}

	var execRes types.ExecutionResult
	err = json.Unmarshal(res, &execRes)

	return execRes.GasUsed, err

}

// Query will do a query request to VM
func (vm *VM) QueryEntryFunction(
	kvStore api.KVStore,
	goApi api.GoAPI,
	querier api.Querier,
	gasLimit uint64,
	payload types.ExecuteEntryFunctionPayload,
) ([]byte, error) {
	bz, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}

	res, err := api.QueryContract(
		kvStore,
		goApi,
		querier,
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
	querier api.Querier,
	gasLimit uint64,
	txHash types.Bytes, // txHash is used for sessionID
	sender types.AccountAddress,
	payload types.ExecuteEntryFunctionPayload,
) (uint64, []types.Event, error) {
	bz, err := json.Marshal(payload)
	if err != nil {
		return 0, nil, err
	}

	res, err := api.ExecuteContract(
		kvStore,
		goApi,
		querier,
		vm.printDebug,
		gasLimit,
		txHash,
		sender,
		bz,
	)

	if err != nil {
		return 0, nil, err
	}

	var execRes types.ExecutionResult
	err = json.Unmarshal(res, &execRes)
	return execRes.GasUsed, execRes.Events, err
}

// Execute calls a given contract.
// TODO: add params and returns
func (vm *VM) ExecuteScript(
	kvStore api.KVStore,
	goApi api.GoAPI,
	querier api.Querier,
	gasLimit uint64,
	txHash types.Bytes, // txHash is used for sessionID
	sender types.AccountAddress,
	payload types.ExecuteScriptPayload,
) (uint64, []types.Event, error) {
	bz, err := json.Marshal(payload)
	if err != nil {
		return 0, nil, err
	}

	res, err := api.ExecuteScript(
		kvStore,
		goApi,
		querier,
		vm.printDebug,
		gasLimit,
		txHash,
		sender,
		bz,
	)

	if err != nil {
		return 0, nil, err
	}

	var execRes types.ExecutionResult
	err = json.Unmarshal(res, &execRes)
	return execRes.GasUsed, execRes.Events, err
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
