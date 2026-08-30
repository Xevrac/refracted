using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TunnelNetworkAllowedChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TunnelNetworkAllowedChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TunnelNetworkAllowedChanged)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize array AllowedCategories
            Rts.Serialization.Reference.Write(s, value.AllowedCategories, () =>
            {
                s.WriteVarInt32(value.AllowedCategories.Length);
                for(int i = 0 ; i < value.AllowedCategories.Length ; ++i)
                {
                    s.Write(value.AllowedCategories[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TunnelNetworkAllowedChanged)) as Rts.CnC.Messages.Client.TunnelNetworkAllowedChanged;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize array AllowedCategories
            Rts.Serialization.Reference.Read(s, out value.AllowedCategories, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
