module Sdkwork
  module BackendSdk
    module Models
      class CreateServerRequest
              attr_accessor :name, :host, :ssh_port, :ssh_user, :ssh_key_path, :description

              def initialize(attributes = {})
                attributes = (attributes || {}).transform_keys(&:to_s)
                @name = attributes['name']
                @host = attributes['host']
                @ssh_port = attributes['sshPort']
                @ssh_user = attributes['sshUser']
                @ssh_key_path = attributes['sshKeyPath']
                @description = attributes['description']
              end

              def self.from_hash(data)
                return nil if data.nil?

                new(data)
              end

              def to_hash
                {
                  'name' => @name,
                  'host' => @host,
                  'sshPort' => @ssh_port,
                  'sshUser' => @ssh_user,
                  'sshKeyPath' => @ssh_key_path,
                  'description' => @description,
                }
              end
            end
    end
  end
end
