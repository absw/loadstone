#!groovy

properties([[
    $class: 'BuildDiscarderProperty',
    strategy: [
        $class: 'LogRotator',
        artifactDaysToKeepStr: '',
        artifactNumToKeepStr: '',
        daysToKeepStr: '',
        numToKeepStr: env.BRANCH_NAME ==~ /master/ ? '10' : '2']
    ]]
)

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
                    sh 'cargo check'
                }
            }
        }
        stage('Linting') {
            steps {
                container('rust') {
                    sh 'cargo clippy'
                }
            }
        }
        stage('Documentation') {
            steps {
                container('rust') {
                    sh 'cargo d'
                }
            }
        }
        stage('Build binary') {
            when { tag "*" }
            steps {
                container('rust') {
                    echo 'Building binary only on master branch...'
                    sh 'cargo install cargo-binutils'
                    sh 'rustup component add llvm-tools-preview'
                    sh 'cargo gen_loadstone'

                    echo 'Archiving Artifacts'
                    archiveArtifacts artifacts: 'loadstone.bin'
                    sh 'rm -f target/thumbv7em-none-eabihf/doc/.lock'
                    archiveArtifacts artifacts: 'target/thumbv7em-none-eabihf/doc/**'
                    publishHTML (target: [
                        allowMissing: false,
                        alwaysLinkToLastBuild: false,
                        keepAll: true,
                        reportDir: 'target/thumbv7em-none-eabihf/doc/',
                        reportFiles: 'secure_bootloader/index.html',
                        reportName: "Loadstone Documentation"
                    ])
                }
            }
        }
    }
}
