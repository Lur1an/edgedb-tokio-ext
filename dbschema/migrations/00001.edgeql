CREATE MIGRATION m1337sgxdkoqaamvzbqtp3kyffqhxldxuojr3hxkpowov37k5tsgzq
    ONTO initial
{
  CREATE TYPE default::Organization {
      CREATE REQUIRED PROPERTY name: std::str;
  };
  CREATE TYPE default::User {
      CREATE LINK org: default::Organization;
      CREATE REQUIRED PROPERTY age: std::int64 {
          CREATE CONSTRAINT std::min_value(1);
      };
      CREATE REQUIRED PROPERTY name: tuple<first: std::str, last: std::str>;
  };
  ALTER TYPE default::Organization {
      CREATE MULTI LINK users := (.<org[IS default::User]);
  };
};
