#!/bin/bash
export NVM_DIR=$HOME/.nvm
source $NVM_DIR/nvm.sh
nvm use 16
pushd client/
npm run build
popd
