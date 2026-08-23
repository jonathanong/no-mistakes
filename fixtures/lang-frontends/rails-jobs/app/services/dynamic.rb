module Services
  class Dynamic
    def call(name)
      name.constantize
      CONST_JOB.constantize.perform_later
      "QuotedConst".constantize
      'AlsoQuoted'.constantize
    end
  end
end

