import hudson.tasks.test.AbstractTestResultAction
import hudson.model.Actionable
import hudson.tasks.junit.CaseResult

pipeline {
    agent none
    parameters {
        booleanParam(name: 'PUBLISH', defaultValue: false, description: 'Set to true to publish a new version to crates.io')
        choice(name: 'PUBLISH_BUMP', choices: ['patch', 'minor', 'major'], description: 'What to bump when publishing') }
    options {
        buildDiscarder(logRotator(numToKeepStr: '50'))
        disableConcurrentBuilds()
    }
    environment {
        GITHUB_TOKEN = credentials('githubrelease')

        REPOSITORY_OWNER = 'feenkcom'
        REPOSITORY_NAME = 'shared-library-builder'

        MACOS_INTEL_TARGET = 'x86_64-apple-darwin'
        MACOS_M1_TARGET = 'aarch64-apple-darwin'

        WINDOWS_SERVER_NAME = 'daffy-duck'
        WINDOWS_AMD64_TARGET = 'x86_64-pc-windows-msvc'

        LINUX_SERVER_NAME = 'mickey-mouse'
        LINUX_AMD64_TARGET = 'x86_64-unknown-linux-gnu'
    }

    stages {
        stage ('Parallel build') {
            parallel {
                stage ('MacOS x86_64') {
                    agent {
                        label "${MACOS_INTEL_TARGET}"
                    }
                    environment {
                        TARGET = "${MACOS_INTEL_TARGET}"
                        PATH = "$HOME/.cargo/bin:/usr/local/bin/:$PATH"
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'git clean -fdx'
                        sh "cargo build --release --all-features"
                        //sh "cargo test --all-features -- --nocapture --test-threads=1"
                        sh "cargo clippy --manifest-path Cargo.toml -- -W clippy::style -W clippy::correctness -W clippy::complexity -W clippy::perf"
                    }
                }
                stage ('MacOS M1') {
                    agent {
                        label "${MACOS_M1_TARGET}"
                    }

                    environment {
                        TARGET = "${MACOS_M1_TARGET}"
                        PATH = "$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'git clean -fdx'
                        sh "cargo build --release --all-features"
                        //sh "cargo test --all-features -- --nocapture --test-threads=1"
                        sh "cargo clippy --manifest-path Cargo.toml -- -W clippy::style -W clippy::correctness -W clippy::complexity -W clippy::perf"
                    }
                }
                stage ('Linux x86_64') {
                    agent {
                        label "${LINUX_AMD64_TARGET}-${LINUX_SERVER_NAME}"
                    }
                    environment {
                        TARGET = "${LINUX_AMD64_TARGET}"
                        PATH = "$HOME/.cargo/bin:$PATH"
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'git clean -fdx'
                        sh "cargo build --release --all-features"
                        //sh "cargo test --all-features -- --nocapture --test-threads=1"
                        sh "cargo clippy --manifest-path Cargo.toml -- -W clippy::style -W clippy::correctness -W clippy::complexity -W clippy::perf"
                    }
                }
                stage ('Windows x86_64') {
                    agent {
                        label "${WINDOWS_AMD64_TARGET}-${WINDOWS_SERVER_NAME}"
                    }

                    environment {
                        TARGET = "${WINDOWS_AMD64_TARGET}"
                        LLVM_HOME = 'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\VC\\Tools\\Llvm\\x64'
                        LIBCLANG_PATH = "${LLVM_HOME}\\bin"
                        CMAKE_PATH = 'C:\\Program Files\\CMake\\bin'
                        MSBUILD_PATH = 'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\MSBuild\\Current\\Bin'
                        CARGO_HOME = "C:\\.cargo"
                        CARGO_PATH = "${CARGO_HOME}\\bin"
                        PATH = "${CARGO_PATH};${LIBCLANG_PATH};${MSBUILD_PATH};${CMAKE_PATH};$PATH"
                    }

                    steps {
                        powershell 'Remove-Item -Force -Recurse -Path target -ErrorAction Ignore'
                        powershell 'git clean -fdx'
                        powershell "cargo build --release --all-features "
                        //powershell "cargo test --all-features -- --nocapture --test-threads=1"
                        powershell "cargo clippy --manifest-path Cargo.toml -- -W clippy::style -W clippy::correctness -W clippy::complexity -W clippy::perf"
                    }
                }
            }
        }
        stage ('Publish on crates.io') {
            agent {
                label "${MACOS_M1_TARGET}"
            }
            environment {
                TARGET = "${MACOS_M1_TARGET}"
            }
            when {
                expression {
                    (currentBuild.result == null || currentBuild.result == 'SUCCESS') && env.BRANCH_NAME.toString().equals('main') && params.PUBLISH
                }
            }
            steps {
                sh "cargo publish --dry-run"
                sh "cargo publish"
            }
        }

        stage ('Release on github') {
            agent {
                label "${MACOS_M1_TARGET}"
            }
            environment {
                TARGET = "${MACOS_M1_TARGET}"
            }
            when {
                expression {
                    (currentBuild.result == null || currentBuild.result == 'SUCCESS') && env.BRANCH_NAME.toString().equals('main')
                }
            }
            steps {
                sh "curl -o feenk-releaser -LsS https://github.com/feenkcom/releaser-rs/releases/latest/download/feenk-releaser-${TARGET}"
                sh "chmod +x feenk-releaser"

                sh """
                ./feenk-releaser \
                    --owner ${REPOSITORY_OWNER} \
                    --repo ${REPOSITORY_NAME} \
                    --token GITHUB_TOKEN \
                    --bump-${params.PUBLISH_BUMP} \
                    --auto-accept """
            }
        }
    }
}
