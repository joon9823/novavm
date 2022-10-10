package types

// BuildConfig is a configuration set to compile move package
type BuildConfig struct {
	DevMode            bool
	TestMode           bool
	GenerateDocs       bool
	GenerateABIs       bool
	InstallDir         []byte
	ForceRecompilation bool
	FetchDepsOnly      bool
}

// DefaultBuildConfig returns with all-false set (except PackagePath which is set to current(.)) BuildConfig
func DefaultBuildConfig() BuildConfig {
	return BuildConfig{
		InstallDir: []byte(""),
		// else all set to false
	}
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
		bc.InstallDir = []byte(dir)
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

// TestConfig is a configuration set to test move package
type TestConfig struct {
	InstructionExecutionBound uint64
	Filter                    []byte
	List                      bool
	NumThreads                uint
	ReportStatistics          bool
	ReportStorageOnError      bool
	IgnoreCompileWarnings     bool
	CheckStacklessVM          bool
	VerboseMode               bool
	ComputeCoverage           bool
}

const (
	DefaultInstructionExecutionBound = 200_000 // twice of aptos default
	DefaultNumThreads                = 8       // same with move default
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

type CoverageSummary struct {
	Function  bool
	OutputCSV bool
}

type CoverageSource string

type CoverageBytecode string

type DisassembleOption struct {
	Interactive        bool
	PackageName        string
	ModuleOrScriptName string
}

type ProveOption struct {
	TargetFilter string
	ForTest      bool
	Options      string
}

type DocgenOption struct {
	SectionLevelStart          uint
	ExcludePrivateFun          bool
	ExcludeSpecs               bool
	IndependentSpecs           bool
	ExcludeImpl                bool
	TocDeps                    uint
	NoCollapsedSections        bool
	OutputDirectory            string
	Template                   []string
	ReferencesFile             string
	IncludeDepDiagrams         bool
	IncludeCallDiagrams        bool
	CompileRelativeToOutputDir bool
}

type ExperimentalCommand_ReadWriteSet struct {
	ModuleFile string
	FunName    string
	Signers    string
	Args       string
	TypeArgs   string
	Concretize uint8
}

type ExperimentalOption struct {
	StorageDir string
	Cmd        interface{}
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
