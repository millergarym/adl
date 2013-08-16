#include "test.h"

namespace ADL {
namespace test {

A::A()
    : f_int(0)
    , f_bool(false)
{
}

A::A(
    const int16_t & f_int_,
    const std::string & f_string_,
    const bool & f_bool_
    )
    : f_int(f_int_)
    , f_string(f_string_)
    , f_bool(f_bool_)
{
}

bool
operator<( const A &a, const A &b )
{
    if( a.f_int < b.f_int ) return true;
    if( b.f_int < a.f_int ) return false;
    if( a.f_string < b.f_string ) return true;
    if( b.f_string < a.f_string ) return false;
    if( a.f_bool < b.f_bool ) return true;
    if( b.f_bool < a.f_bool ) return false;
    return false;
}

bool
operator==( const A &a, const A &b )
{
    return
        a.f_int == b.f_int &&
        a.f_string == b.f_string &&
        a.f_bool == b.f_bool ;
}

U::U()
    : d_(F_INT), p_(new int16_t(0))
{
}

U U::mk_f_int( const int16_t & v )
{
    return U( F_INT, new int16_t(v) );
}

U U::mk_f_string( const std::string & v )
{
    return U( F_STRING, new std::string(v) );
}

U::U( const U & v )
    : d_(v.d_), p_(copy(v.d_,v.p_))
{
}

U::~U()
{
    free(d_,p_);
}

U & U::operator=( const U & o )
{
    free(d_,p_);
    d_ = o.d_;
    p_ = copy( o.d_, o.p_ );
}

const int16_t & U::set_f_int(const int16_t &v)
{
    if( d_ == F_INT )
    {
        *(int16_t *)p_ = v;
    }
    else
    {
        free(d_,p_);
        d_ = F_INT;
        p_ = new int16_t(v);
    }
    return *(int16_t *)p_;
}

const std::string & U::set_f_string(const std::string &v)
{
    if( d_ == F_STRING )
    {
        *(std::string *)p_ = v;
    }
    else
    {
        free(d_,p_);
        d_ = F_STRING;
        p_ = new std::string(v);
    }
    return *(std::string *)p_;
}

U::U(DiscType d, void *p)
    : d_(d), p_(p)
{
}

void U::free(DiscType d, void *p)
{
    switch( d )
    {
        case F_INT: delete (int16_t *)p;
        case F_STRING: delete (std::string *)p;
    }
}

void * U::copy( DiscType d, void *p )
{
    switch( d )
    {
        case F_INT: return new int16_t(*(int16_t *)p);
        case F_STRING: return new std::string(*(std::string *)p);
    }
}

bool
operator<( const U &a, const U &b )
{
    if( a.d() < b.d() ) return true;
    if( b.d() < a.d()) return false;
    switch( a.d() )
    {
        case U::F_INT: return a.f_int() < b.f_int();
        case U::F_STRING: return a.f_string() < b.f_string();
    }
}

bool
operator==( const U &a, const U &b )
{
    if( a.d() != b.d() ) return false;
    switch( a.d() )
    {
        case U::F_INT: return a.f_int() == b.f_int();
        case U::F_STRING: return a.f_string() == b.f_string();
    }
}

}
}