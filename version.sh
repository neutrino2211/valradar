#! /bin/bash
#######################################################################################
# Bash function get_toml_value() to get the value of a key in a section of a TOML file.
# Versioned here: https://gist.github.com/kwmiebach/e42dc4a43d5a2a0f2c3fdc41620747ab
# Call it like this:
# value=$(get_toml_value "./conf/config.toml" "server_b" "domain")
# Result should be "my123.example.com" in the case of this example file:
# ---------------------------
# [server_a]
# proto = "https"
# domain = "test.example.net"
# access_token = "*******123**********"
# 
# [server_b]
# proto = "http"
# domain = "my123.example.com"
# access_token = "*******123**********"
# ---------------------------
# Does not work for other more complex TOML files.

get_toml_value() {
    # Takes three parameters:
    # - TOML file path ($1)
    # - section ($2)
    # - the key ($3)
    # 
    # It first gets the section using the get_section function
    # Then it finds the key within that section
    # using grep and cut.

    local file="$1"
    local section="$2"
    local key="$3"

    get_section() {
        # Function to get the section from a TOML file
        # Takes two parameters:
        # - TOML file path ($1)
        # - section name ($2)
        # 
        # It uses sed to find the section
        # A section is terminated by a line with [ in pos 0 or the end of file.

        local file="$1"
        local section="$2"

        sed -n "/^\[$section\]/,/^\[/p" "$file" | sed '$d'
    }
        
    get_section "$file" "$section" | grep "^$key " | cut -d "=" -f2- | tr -d ' "'
}  
# End Function get_toml_value()
#####################################################################################

version=$(get_toml_value "Cargo.toml" "package" "version")
echo $version
