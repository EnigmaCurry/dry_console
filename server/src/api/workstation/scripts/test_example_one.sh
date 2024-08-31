# # A simple example test for remote process execution and markdown description.
#
#  * This description is extracted from the top of the scrtipt file.
#  * The source accepts most Markdown features:
#    * [🌴](/apps)
#
# ## The description continues until the first non-comment line.
# # one
# ## two
# ### three
# #### four
# ##### five
# ###### six

# This is just a regular comment.
# Count to 100
(
    set -e
    echo "Hii" >/dev/stderr
    for i in $(seq 100); do
        echo $i
        sleep 0.1
    done
)
