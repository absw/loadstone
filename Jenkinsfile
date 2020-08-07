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
        stage('Build binary') {
            when { branch "master" }
            steps {
                container('rust') {
                        echo 'Building binary only because this commit is tagged...'
                        sh './cargo_emb build'
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
    }
    post {
        always {
            archiveArtifacts artifacts: '*secure_bootloader', onlyIfSuccessful: true
        }
    }
}