using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestFWAAttack
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestFWAAttack); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestFWAAttack)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize AirfieldId
            s.Write(value.AirfieldId);
            //  Serialize TarPlayerId
            s.Write(value.TarPlayerId);
            //  Serialize TarEntityId
            s.Write(value.TarEntityId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestFWAAttack)) as Rts.CnC.Messages.Client.RequestFWAAttack;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize AirfieldId
            s.Read(out value.AirfieldId);
            //  Deserialize TarPlayerId
            s.Read(out value.TarPlayerId);
            //  Deserialize TarEntityId
            s.Read(out value.TarEntityId);

            return value;
        }
        
    }
}
