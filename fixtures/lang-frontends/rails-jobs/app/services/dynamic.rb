module Services
  class Dynamic
    def call(name)
      name.constantize
    end
  end
end
