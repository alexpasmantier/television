#!/bin/sh
PREFIX={prefix}
VERSION={version}
NAME={name}
EXEC={exec}

if [ $1 == "uninstall" ]
then
    echo "Uninstalling $NAME"
    rm -rf $PREFIX/bin/$EXEC
    exit
fi

if [ $1 == "help" ]
then
  echo "\`./install.sh uninstall' to uninstall the program."
  echo "\`./install.sh' or \`./install.sh install' to install the program."
  echo "\`./install.sh' help to show this page."
  exit
fi

echo "Installing $NAME, version: $VERSION, into: $PREFIX/bin/$EXEC"

cp files/$EXEC $PREFIX/bin/
