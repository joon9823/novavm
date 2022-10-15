package types

// BuildConfig is a configuration set to compile move package
type BuildConfig struct {
	// Compile in 'dev' mode. The 'dev-addresses' and 'dev-dependencies' fields will be used if
	// this flag is set. This flag is useful for development of packages that expose named
	// addresses that are not set to a specific value.
	DevMode bool

	// Compile in 'test' mode. The 'dev-addresses' and 'dev-dependencies' fields will be used
	// along with any code in the 'tests' directory.
	TestMode bool

	// Generate documentation for packages
	GenerateDocs bool

	// Generate ABIs for packages
	GenerateABIs bool

	// Installation directory for compiled artifacts. Defaults to current directory.
	InstallDir string

	// Force recompilation of all packages
	ForceRecompilation bool

	// Only fetch dependency repos to MOVE_HOME
	FetchDepsOnly bool

	// Skip fetching latest git dependencies
	SkipFetchLatestGitDeps bool
}

// DefaultBuildConfig returns with all-false set (except PackagePath which is set to current(.)) BuildConfig
func DefaultBuildConfig() BuildConfig {
	return BuildConfig{}
}

// NewBuildConfig returns newly create BuildConfig. unset values stays default, not unset
func NewBuildConfig(options ...func(*BuildConfig)) BuildConfig {
	bc := DefaultBuildConfig()
	for _, opt := range options {
		opt(&bc)
	}
	return bc
}

func WithInstallDir(dir string) func(*BuildConfig) {
	return func(bc *BuildConfig) {
		bc.InstallDir = dir
	}
}

func WithDevMode() func(*BuildConfig) {
	return func(bc *BuildConfig) {
		bc.DevMode = true
	}
}

func WithTestMode() func(*BuildConfig) {
	return func(bc *BuildConfig) {
		bc.TestMode = true
	}
}

func WithGenerateDocs() func(*BuildConfig) {
	return func(bc *BuildConfig) {
		bc.GenerateDocs = true
	}
}

func WithGenerateABIs() func(*BuildConfig) {
	return func(bc *BuildConfig) {
		bc.GenerateABIs = true
	}
}

func WithForceRecompiliation() func(*BuildConfig) {
	return func(bc *BuildConfig) {
		bc.ForceRecompilation = true
	}
}

func WithFetchDepsOnly() func(*BuildConfig) {
	return func(bc *BuildConfig) {
		bc.FetchDepsOnly = true
	}
}

func WithSkipFetchLatestGitDeps() func(*BuildConfig) {
	return func(bc *BuildConfig) {
		bc.SkipFetchLatestGitDeps = true
	}
}

// TestConfig is a configuration set to test move package
type TestConfig struct {
	// Bound the number of instructions that can be executed by any one test.
	// set 0 to no-boundary
	InstructionExecutionBound uint64

	// A filter string to determine which unit tests to run. A unit test will be run only if it
	// contains this string in its fully qualified (<addr>::<module_name>::<fn_name>) name.
	Filter []byte

	// List all tests
	List bool

	// Number of threads to use for running tests.
	NumThreads uint

	// Report test statistics at the end of testing
	ReportStatistics bool

	// Show the storage state at the end of execution of a failing test
	ReportStorageOnError bool

	// Ignore compiler's warning, and continue run tests
	IgnoreCompileWarnings bool

	// Use the stackless bytecode interpreter to run the tests and cross check its results with
	// the execution result from Move VM.
	CheckStacklessVM bool

	// Verbose mode
	VerboseMode bool

	// Collect coverage information for later use with the various `package coverage` subcommands
	ComputeCoverage bool
}

const (
	DefaultInstructionExecutionBound = 200_000
	DefaultNumThreads                = 8
)

// DefaultTestConfig returns TestConfig with default value
func DefaultTestConfig() TestConfig {
	return TestConfig{
		InstructionExecutionBound: DefaultInstructionExecutionBound,
		NumThreads:                DefaultNumThreads,
		// else all set to false
	}
}

// NewTestConfig returns newly create TestConfig. unset values stays default, not unset
func NewTestConfig(options ...func(*TestConfig)) TestConfig {
	tc := DefaultTestConfig()
	for _, opt := range options {
		opt(&tc)
	}
	return tc
}

func WithInstructionExecutionBound(bound uint64) func(*TestConfig) {
	return func(tc *TestConfig) {
		tc.InstructionExecutionBound = bound
	}
}

func WithFilter(filter string) func(*TestConfig) {
	return func(tc *TestConfig) {
		tc.Filter = []byte(filter)
	}
}

func WithList() func(*TestConfig) {
	return func(tc *TestConfig) {
		tc.List = true
	}
}

func WithNumThreads(n uint) func(*TestConfig) {
	return func(tc *TestConfig) {
		tc.NumThreads = n
	}
}

func WithReportStatistics() func(*TestConfig) {
	return func(tc *TestConfig) {
		tc.ReportStatistics = true
	}
}

func WithReportStorageOnError() func(*TestConfig) {
	return func(tc *TestConfig) {
		tc.ReportStorageOnError = true
	}
}

func WithIgnoreCompileWarnings() func(*TestConfig) {
	return func(tc *TestConfig) {
		tc.IgnoreCompileWarnings = true
	}
}

func WithCheckStacklessVM() func(*TestConfig) {
	return func(tc *TestConfig) {
		tc.CheckStacklessVM = true
	}
}

func WithVerboseTestConfig() func(*TestConfig) {
	return func(tc *TestConfig) {
		tc.VerboseMode = true
	}
}

func WithComputeCoverage() func(*TestConfig) {
	return func(tc *TestConfig) {
		tc.ComputeCoverage = true
	}
}

type NovaCompilerArgument struct {
	PackagePath string
	Verbose     bool
	BuildConfig BuildConfig
}

func NewNovaCompilerArgument(packagePath string, verbose bool, buildConfig BuildConfig) NovaCompilerArgument {
	return NovaCompilerArgument{packagePath, verbose, buildConfig}
}

func NewNovaCompilerArgumentWithBuildOption(packagePath string, verbose bool, options ...func(*BuildConfig)) NovaCompilerArgument {
	return NovaCompilerArgument{packagePath, verbose, NewBuildConfig(options...)}
}
