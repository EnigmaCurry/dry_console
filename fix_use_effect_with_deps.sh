#!/bin/bash

# https://yew.rs/docs/migration-guides/yew/from-0_20_0-to-0_21_0#automated-refactor

alias sg=ast-grep

sg --pattern 'use_effect_with_deps($CALLBACK,$$$DEPENDENCIES)' --rewrite 'use_effect_with($$$DEPENDENCIES, $CALLBACK)' -l rs -i
sg --pattern 'use_effect_with($DEPENDENCIES,,$$$CALLBACK)' --rewrite 'use_effect_with($DEPENDENCIES,$$$CALLBACK)' -l rs -i

sg --pattern 'use_callback($CALLBACK,$$$DEPENDENCIES)' --rewrite 'use_callback($$$DEPENDENCIES, $CALLBACK)' -l rs -i
sg --pattern 'use_callback($DEPENDENCIES,,$$$CALLBACK)' --rewrite 'use_callback($DEPENDENCIES,$$$CALLBACK)' -l rs -i

sg --pattern 'use_memo($CALLBACK,$$$DEPENDENCIES)' --rewrite 'use_memo($$$DEPENDENCIES, $CALLBACK)' -l rs -i
sg --pattern 'use_memo($DEPENDENCIES,,$$$CALLBACK)' --rewrite 'use_memo($DEPENDENCIES,$$$CALLBACK)' -l rs -i

sg --pattern 'use_future_with_deps($CALLBACK,$$$DEPENDENCIES)' --rewrite 'use_effect_with($$$DEPENDENCIES, $CALLBACK)' -l rs -i
sg --pattern 'use_future_with($DEPENDENCIES,,$$$CALLBACK)' --rewrite 'use_effect_with($DEPENDENCIES,$$$CALLBACK)' -l rs -i

sg --pattern 'use_transitive_state!($DEPENDENCIES,,$$$CALLBACK)' --rewrite 'use_transitive_state!($DEPENDENCIES,$$$CALLBACK)' -l rs -i
sg --pattern 'use_transitive_state!($DEPENDENCIES,,$$$CALLBACK)' --rewrite 'use_transitive_state!($DEPENDENCIES,$$$CALLBACK)' -l rs -i

sg --pattern 'use_prepared_state!($DEPENDENCIES,,$$$CALLBACK)' --rewrite 'use_prepared_state!($DEPENDENCIES,$$$CALLBACK)' -l rs -i
sg --pattern 'use_prepared_state!($DEPENDENCIES,,$$$CALLBACK)' --rewrite 'use_prepared_state!($DEPENDENCIES,$$$CALLBACK)' -l rs -i
