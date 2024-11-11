{...}: {
  perSystem = {pkgs, ...}: {
    nci.projects."television" = {
      path = ./.;
      export = false;
    };
    # configure crates
    nci.crates = {
      "television" = {
        export = true;
      };
      "television-fuzzy" = {};
      "television-derive" = {};
      "television-channels" = {};
      "television-previewers" = {};
      "television-utils" = {};
    };
  };
}
