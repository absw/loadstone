pipeline {
    agent {
        kubernetes {
      yaml """
apiVersion: v1
kind: Pod
metadata:
  labels:
spec:
  containers:
  - name: rust
    image: rustlang/rust:nightly-stretch
    command:
    - cat
    tty: true
    resources:
      limits:
        cpu: 1.5
        memory: 2Gi
      requests:
        cpu: 300m
        memory: 512Mi
"""
        }
  }
  stages {
        stage('Test') {
            steps {
                container('rust') {
                    sh 'cargo test'
                }
            }
        }
        stage('Check Build') {
            steps {
                container('rust') {
                    sh 'rustup target add thumbv7em-none-eabihf'
                    sh './cargo_emb check'
                }
            }
        }
        stage('Static analysis') {
            steps {
                container('rust') {
                    sh 'cargo clippy'
                }
            }
        }
        stage('Documentation') {
            steps {
                container('rust') {
                    sh './cargo_emb doc'
                }
            }
        }
        stage('Build binary') {
            when { branch "PublishArtifact" }
            steps {
                container('rust') {
                    echo 'Building binary only on master branch...'
                    sh 'cargo install cargo-binutils'
                    sh 'rustup component add llvm-tools-preview'
                    sh 'cargo objcopy --bin secure_bootloader --release --target thumbv7em-none-eabihf --features "stm32f412" -- -O binary bootloader.bin'
                    archiveArtifacts artifacts: '**/bootloader.bin'
                    archiveArtifacts artifacts: '**/target/thumbv7em-none-eabihf/doc**'
                }
            }
        }
    }
}