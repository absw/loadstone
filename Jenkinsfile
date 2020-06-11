#!groovy
podTemplate(
 ) {
    node(POD_LABEL) {
        stage('Checkout SCM') {
            checkout([
                $class: "GitSCM",
                branches: scm.branches,
                extensions: scm.extensions + [
                    [$class: "GitLFSPull"]
                ],
                userRemoteConfigs: scm.userRemoteConfigs
            ])
        }
        stage('Say Hi!') {
            sh 'echo Hello World'
        }
    }
}