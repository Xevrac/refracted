using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestEnterTunnelAccess
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestEnterTunnelAccess); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestEnterTunnelAccess)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize TunnelAccessId
            s.Write(value.TunnelAccessId);
            //  Serialize array EnteringUnitIds
            Rts.Serialization.Reference.Write(s, value.EnteringUnitIds, () =>
            {
                s.WriteVarInt32(value.EnteringUnitIds.Length);
                for(int i = 0 ; i < value.EnteringUnitIds.Length ; ++i)
                {
                    s.Write(value.EnteringUnitIds[i]);
                }
            });
            //  Serialize ModifierFlags
            s.WriteEnum(value.ModifierFlags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestEnterTunnelAccess)) as Rts.CnC.Messages.Client.RequestEnterTunnelAccess;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize TunnelAccessId
            s.Read(out value.TunnelAccessId);
            //  Deserialize array EnteringUnitIds
            Rts.Serialization.Reference.Read(s, out value.EnteringUnitIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize ModifierFlags
            s.ReadEnum(out value.ModifierFlags);

            return value;
        }
        
    }
}
